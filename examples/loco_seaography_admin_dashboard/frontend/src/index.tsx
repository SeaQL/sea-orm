import ReactDOM from 'react-dom/client';
import { Admin, Resource, ListGuesser, ShowGuesser } from 'react-admin';
import { dataProvider } from './dataProvider';

ReactDOM.createRoot(document.getElementById('root')!).render(
    <Admin dataProvider={dataProvider}>
        <Resource name="users" list={ListGuesser} show={ShowGuesser} />
    </Admin>
);
